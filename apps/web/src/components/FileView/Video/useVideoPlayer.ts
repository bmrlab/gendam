import { ExtractAssetObject } from '@/Explorer/types'
import useDebouncedCallback from '@/hooks/useDebouncedCallback'
import { useCurrentLibrary } from '@/lib/library'
import { client } from '@/lib/rspc'
import { timeToSeconds } from '@/lib/utils'
import muxjs from 'mux.js'
import { useEffect, useRef } from 'react'
import videojs from 'video.js'
import type Player from 'video.js/dist/types/player'

const VIDEO_TS_SIZE = 10

export const useVideoPlayer = (assetObject: ExtractAssetObject<'video'>, currentTime?: number) => {
  const videoRef = useRef<HTMLDivElement>(null)
  const videoElementRef = useRef<HTMLVideoElement | null>(null)
  const currentLibrary = useCurrentLibrary()
  const playerRef = useRef<Player | null>(null)
  const mediaSourceRef = useRef<MediaSource>(new MediaSource())
  const sourceBufferRef = useRef<SourceBuffer | null>(null)
  const transmuxerRef = useRef<muxjs.mp4.Transmuxer | null>(null)
  const segmentsRef = useRef<number[]>([])

  const debounceSeeking = useDebouncedCallback((seekTimeSegment: number) => {
    if (segmentsRef.current.includes(seekTimeSegment)) {
      let index = segmentsRef.current.indexOf(seekTimeSegment)
      segmentsRef.current = segmentsRef.current.slice(index)
    }
  }, 100)

  const handleUpdateend = async () => {
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
          hash: assetObject.hash,
          index: item,
          size: VIDEO_TS_SIZE,
        },
      ])
      transmuxerRef.current?.push(new Uint8Array(res.data))
      transmuxerRef.current?.flush()
    }
  }

  const loadVideoTS = async () => {
    const mediaData = assetObject.mediaData
    if (!mediaData) return
    if (!videoRef.current || !videoElementRef.current) {
      return
    }

    segmentsRef.current = Array.from(new Array(Math.ceil(mediaData.duration / VIDEO_TS_SIZE))).map((_, i) => i)

    mediaSourceRef.current = new MediaSource()
    ;(videoElementRef.current.children[0] as HTMLVideoElement).src = URL.createObjectURL(mediaSourceRef.current)

    mediaSourceRef.current.addEventListener('sourceopen', async () => {
      sourceBufferRef.current = mediaSourceRef.current.addSourceBuffer('video/mp4; codecs="avc1.64001e, mp4a.40.2"')

      transmuxerRef.current = new muxjs.mp4.Transmuxer()

      transmuxerRef.current.on('data', (segment) => {
        const data = new Uint8Array(segment.initSegment.byteLength + segment.data.byteLength)
        data.set(segment.initSegment, 0)
        data.set(segment.data, segment.initSegment.byteLength)
        sourceBufferRef.current?.appendBuffer(data)
      })

      sourceBufferRef.current.addEventListener('updateend', handleUpdateend)

      let item = segmentsRef.current.shift()
      if (typeof item !== 'undefined') {
        const res = await client.query([
          'video.player.video_ts',
          {
            hash: assetObject.hash,
            index: item,
            size: VIDEO_TS_SIZE,
          },
        ])
        transmuxerRef.current.push(new Uint8Array(res.data))
        transmuxerRef.current.flush()
      }
    })
  }

  const loadVideo = async () => {
    if (!playerRef.current) {
      return
    }

    const mediaData = assetObject.mediaData

    if (!mediaData) return

    const player = playerRef.current
    player.duration = () => mediaData.duration
    player.poster(currentLibrary.getThumbnailSrc(assetObject.hash, 'video'))

    const src = currentLibrary.getFileSrc(assetObject.hash)
    if (assetObject.mimeType?.includes('mp4')) {
      player.src({ type: 'video/mp4', src })
    } else {
      loadVideoTS()
    }
  }

  useEffect(() => {
    if (!videoRef.current) return

    if (!playerRef.current) {
      const videoElement = document.createElement('video-js')
      videoElement.classList.add('vjs-big-play-centered', 'max-h-full', 'max-w-full', 'w-full', 'h-full')
      videoRef.current.appendChild(videoElement)
      videoElementRef.current = videoElement as HTMLVideoElement

      const player = videojs(
        videoElement,
        {
          controls: true,
          autoPlay: false,
          loop: false,
          muted: true,
          noSupportedMessage: 'This video cannot be played, please try again later',
          controlBar: {
            fullscreenToggle: false,
            pictureInPictureToggle: false,
          },
        },
        () => {
          playerRef.current = player
          // 监听
          if (videoRef.current && playerRef.current) {
            playerRef.current.on('seeking', async () => {
              const tooltipElement = document.querySelector('.vjs-time-tooltip')
              if (tooltipElement) {
                const innerHTML = tooltipElement.innerHTML
                const seekTime = timeToSeconds(innerHTML)
                const seekTimeSegment = Math.ceil(seekTime / VIDEO_TS_SIZE) - 1
                debounceSeeking(seekTimeSegment)
              }
            })
          }
          loadVideo()
        },
      )
    }

    return () => {
      if (videoElementRef.current) videoRef.current?.removeChild(videoElementRef.current)
      if (playerRef.current && !playerRef.current.isDisposed()) {
        playerRef.current.dispose()
        playerRef.current = null
      }
      mediaSourceRef.current = new MediaSource()
      sourceBufferRef.current = null
      transmuxerRef.current = null
      segmentsRef.current = []
      videoElementRef.current = null
    }
  }, [playerRef, assetObject, videoRef])

  useEffect(() => {
    return () => {
      if (transmuxerRef.current) {
        transmuxerRef.current?.off('data')
        transmuxerRef.current?.off('done')
        transmuxerRef.current = null
      }
      segmentsRef.current = []
      mediaSourceRef.current = new MediaSource()
      sourceBufferRef.current = null
    }
  }, [])

  useEffect(() => {
    if (videoElementRef.current && currentTime) {
      videoElementRef.current.currentTime = Math.floor(currentTime / 1e3)
    }
  }, [videoElementRef.current, currentTime])

  return videoRef
}
