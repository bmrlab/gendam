'use client'

import { useCurrentLibrary } from '@/lib/library'
import { useWavesurfer } from '@wavesurfer/react'
import classNames from 'classnames'
import { useEffect, useRef } from 'react'

export default function Audio({ hash, duration, className }: { hash: string; duration?: number; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  const containerRef = useRef<HTMLDivElement>(null)

  const { wavesurfer, isPlaying, currentTime } = useWavesurfer({
    container: containerRef,
    waveColor: 'rgb(208, 213, 228)',
    progressColor: 'rgb(4, 63, 251)',
    barWidth: 2,
    dragToSeek: true,
    normalize: true,
    cursorWidth: 0,
    barGap: 0,
    height: containerRef.current?.clientHeight ?? 0,
    url: currentLibrary.getFileSrc(hash),
  })

  useEffect(() => {
    const audioUrl = currentLibrary.getFileSrc(hash)
    const waveformUrl = currentLibrary.getAudioPreviewSrc(hash)

    ;(async () => {
      try {
        if (!waveformUrl) throw new Error('waveform not found')

        const response = await fetch(waveformUrl)
        const jsonData: number[] = await response.json()

        if (!jsonData || jsonData.length === 0) {
          throw new Error('waveform not found')
        }

        wavesurfer?.load(audioUrl, [jsonData], duration)
      } catch {
        wavesurfer?.load(audioUrl)
      }
    })()
  }, [])

  useEffect(() => {
    wavesurfer?.on('click', () => {
      isPlaying ? wavesurfer?.pause() : wavesurfer?.play()
    })
    wavesurfer?.on('dragstart', () => {
      wavesurfer?.pause()
    })
    wavesurfer?.on('dragend', () => {
      // 防止闪烁
      setTimeout(() => wavesurfer?.play(), 200)
    })
  }, [wavesurfer, isPlaying])

  return (
    <div className={classNames('relative h-full w-full', className)}>
      <div className={classNames('relative z-0 h-full w-full')} ref={containerRef} />
    </div>
  )
}
