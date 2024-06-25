import useDebouncedCallback from '@/hooks/useDebouncedCallback'
import { useCurrentLibrary } from '@/lib/library'
import { client } from '@/lib/rspc'
import { timeToSeconds } from '@/lib/utils'
import muxjs from 'mux.js'
import { MutableRefObject, useEffect, useRef } from 'react'
import videojs from 'video.js'
import type Player from 'video.js/dist/types/player'

export const useVideoPlayer = (hash: string, videoRef: MutableRefObject<HTMLVideoElement | null>) => {
  const currentLibrary = useCurrentLibrary()
  const playerRef = useRef<Player | null>(null)
  const mediaSourceRef = useRef<MediaSource>(new MediaSource())
  const sourceBufferRef = useRef<SourceBuffer | null>(null)
  const transmuxerRef = useRef<muxjs.mp4.Transmuxer | null>(null)
  const segmentsRef = useRef<number[]>([])
  const loadedSegmentsRef = useRef<number[]>([])
  const lastTimeRef = useRef<number>(0)

  const debounceSeeking = useDebouncedCallback((seekTimeSegment: number) => {
    if (segmentsRef.current.includes(seekTimeSegment)) {
      console.log('seeking所在', seekTimeSegment)
      let index = segmentsRef.current.indexOf(seekTimeSegment)
      let old = segmentsRef.current.slice(0, index)
      segmentsRef.current.splice(0, index)
      segmentsRef.current = [...segmentsRef.current, ...old, ...segmentsRef.current]
      console.log('segmentsRef.current', old, segmentsRef.current)
    }
  }, 100)

  const handleUpdateend = async () => {
    transmuxerRef.current?.on('data', (event: any) => {
      try {
        sourceBufferRef.current?.appendBuffer(new Uint8Array(event.data))
      } catch (e) {
        console.warn('sourceBuffer fail to append Buffer: ', e)
      }
      transmuxerRef.current?.off('data')
    })

    if (
      segmentsRef.current.length == 0 &&
      !sourceBufferRef.current?.updating &&
      mediaSourceRef.current.readyState === 'open'
    ) {
      try {
        mediaSourceRef.current.endOfStream()
      } catch (e) {
        console.error('media source endOfStream error:', e)
      }
    }

    let item = segmentsRef.current.shift()
    if (item) {
      const res = await client.query([
        'video.player.video_ts',
        {
          hash: hash,
          index: item,
        },
      ])
      console.log('load segment:', item)
      loadedSegmentsRef.current.push(item)
      transmuxerRef.current?.push(new Uint8Array(res.data))
      transmuxerRef.current?.flush()
    }
  }

  const prepareSourceBuffer = (combined: boolean, outputType: 'audio' | 'video', bytes: Uint8Array) => {
    if (!videoRef.current) {
      return
    }
    mediaSourceRef.current = new MediaSource()
    videoRef.current.src = URL.createObjectURL(mediaSourceRef.current)

    var codecsArray = ['avc1.64001f', 'mp4a.40.2'] // todo 请求获取

    mediaSourceRef.current.addEventListener('sourceopen', function () {
      let buffer
      if (combined) {
        buffer = 'video/mp4;codecs="' + codecsArray.join(',') + '"'
      } else if (outputType === 'video') {
        // 转换为只含视频的mp4
        buffer = 'video/mp4;codecs="' + codecsArray[0] + '"'
      } else {
        // 转换为只含音频的mp4
        buffer = 'audio/mp4;codecs="' + (codecsArray[1] || codecsArray[0]) + '"'
      }

      sourceBufferRef.current = mediaSourceRef.current.addSourceBuffer(buffer)
      sourceBufferRef.current.addEventListener('updateend', handleUpdateend)
      sourceBufferRef.current.appendBuffer(bytes)
    })
  }

  const transferFormat = async (data: number[], hasVideo: boolean, hasAudio: boolean) => {
    const segment = new Uint8Array(data)

    transmuxerRef.current = new muxjs.mp4.Transmuxer()

    // 注意：接收无音频ts文件，OutputType设置为'video'，并且设置combined为 'false',
    // 在监听data事件的时候，控置转换流的类型
    const combined = !!hasVideo && !!hasAudio
    const outputType = hasAudio ? 'audio' : 'video'

    let remuxedSegments: any[] = []
    let remuxedBytesLength = 0
    let remuxedInitSegment: any = null

    transmuxerRef.current?.on('data', function (event) {
      remuxedSegments.push(event)
      remuxedBytesLength += event.data.byteLength
      remuxedInitSegment = event.initSegment
      transmuxerRef.current!.off('data')
    })

    transmuxerRef.current?.on('done', function () {
      let offset = 0
      let bytes = new Uint8Array(remuxedInitSegment.byteLength + remuxedBytesLength)
      bytes.set(remuxedInitSegment, offset)
      offset += remuxedInitSegment.byteLength

      for (let j = 0, i = offset; j < remuxedSegments.length; j++) {
        bytes.set(remuxedSegments[j].data, i)
        i += remuxedSegments[j].byteLength
      }
      remuxedSegments = []
      remuxedBytesLength = 0
      // 解析出转换后的mp4相关信息，与最终转换结果无关
      const vjsParsed = muxjs.mp4.tools.inspect(bytes)
      console.log('transmuxed', vjsParsed)
      // （3.准备资源数据，添加到标签的视频流中
      prepareSourceBuffer(combined, outputType, bytes)
      transmuxerRef.current!.off('done')
    })
    transmuxerRef.current?.push(segment)
    transmuxerRef.current?.flush()
  }

  const onPlayerReady = async (mimeType: string|null, hasVideo: boolean, hasAudio: boolean) => {
    if (mimeType?.includes('mp4')) {
      playerRef.current?.src({ type: 'video/mp4', src: currentLibrary.getFileSrc(hash) })
      return
    }
    const segment = segmentsRef.current.shift()

    const res = await client.query([
      'video.player.video_ts',
      {
        hash: hash,
        index: segment!,
      },
    ])
    loadedSegmentsRef.current.push(segment!)
    transferFormat(res.data, hasVideo, hasAudio)

    // 监听
    if (!!videoRef.current) {
      playerRef.current?.on('seeking', async () => {
        const tooltipElement = document.querySelector('.vjs-time-tooltip')
        if (tooltipElement) {
          const innerHTML = tooltipElement.innerHTML
          const seekTime = timeToSeconds(innerHTML)
          const seekTimeSegment = Math.ceil(seekTime / 10) - 1
          debounceSeeking(seekTimeSegment)
        }
      })
    }
  }

  const init = async () => {
    // 获取时长
    const { duration, mimeType, hasVideo, hasAudio } = await client.query([
      'video.player.video_info',
      {
        hash,
      },
    ])
    segmentsRef.current = Array.from(new Array(Math.ceil(duration / 10))).map((_, i) => i)
    // https://docs.videojs.com/tutorial-options.html
    const option = {
      controls: true,
      autoPlay: true,
      loop: false,
      muted: true,
      noSupportedMessage: 'This video cannot be played, please try again later',
      poster: currentLibrary.getThumbnailSrc(hash),
      controlBar: {
        fullscreenToggle: false,
        pictureInPictureToggle: false,
      },
    }

    playerRef.current = videojs(videoRef.current!, option, () => onPlayerReady(mimeType, hasVideo, hasAudio))

    // 覆盖duration
    playerRef.current.duration = function () {
      return duration
    }

    playerRef.current.play()
  }

  useEffect(() => {
    if (!videoRef.current) return
    init()
    return () => {
      if (playerRef.current) {
        playerRef.current?.dispose()
        playerRef.current?.off('seeking')
      }
      if (transmuxerRef.current) {
        transmuxerRef.current?.off('data')
        transmuxerRef.current?.off('done')
        transmuxerRef.current = null
      }
      segmentsRef.current = []
      loadedSegmentsRef.current = []
      mediaSourceRef.current = new MediaSource()
      sourceBufferRef.current = null
      lastTimeRef.current = 0
    }
  }, [videoRef])
}
