'use client'

import VideoTaskItem from './task-item'
import type { VideoTaskItemProps } from './task-item'
import type { VideoWithTasksResult } from '@/lib/bindings'
import useKeyPress, { KeyType } from '@/hooks/useKeyPress'
import { useBoundStore } from '../_store'
import React from 'react'

type WithSelectProps = {
  items: VideoWithTasksResult[]
} & VideoTaskItemProps

function withSelect<T extends WithSelectProps>(Component: React.ComponentType<T>) {
  // eslint-disable-next-line react/display-name
  return function ({ ...props }: T & WithSelectProps) {
    const videoSelected = useBoundStore.use.videoSelected()
    const addVideoSelected = useBoundStore.use.addVideoSelected()
    const removeVideoSelected = useBoundStore.use.removeVideoSelected()
    const clearVideoSelected = useBoundStore.use.clearVideoSelected()

    const isShiftPressed = useKeyPress(KeyType.Shift)
    const isCommandPressed = useKeyPress(KeyType.Meta)

    const { videoFile } = props as WithSelectProps
    const { assetObject, materializedPath } = videoFile

    const handleClick = async () => {
      // 按住 shift 键，多选视频
      if (videoSelected.length >= 1 && isShiftPressed) {
        let newVideoSelected = [
          ...videoSelected,
          {
            assetObject,
            materializedPath,
          },
        ]

        // FIXME: 这里现在用 assetObject.id 来排序以及圈定上届和下届可能会有问题，因为 id 不连续
        newVideoSelected.sort((a, b) => a.assetObject.id - b.assetObject.id)
        let maxIndex = newVideoSelected[newVideoSelected.length - 1].assetObject.id
        let minIndex = newVideoSelected[0].assetObject.id
        const needAdd = props.items.filter((item) => item.assetObject.id >= minIndex && item.assetObject.id <= maxIndex)
        addVideoSelected(needAdd)
        return
      }

      // 如果按住 command 键，点击已选中的视频，取消选中
      if (videoSelected.some((item) => item.assetObject.id === assetObject.id) && isCommandPressed) {
        removeVideoSelected(assetObject.id)
        return
      }

      // 默认单选视频
      clearVideoSelected()
      addVideoSelected(videoFile)
    }

    const handleRightClick = () => {
      const notSelect = videoSelected.every((item) => item.assetObject.hash !== assetObject.hash)
      if (notSelect) {
        clearVideoSelected()
        addVideoSelected(videoFile)
      }
    }

    const { items, ...newProps } = props
    return <Component {...(newProps as T)} onClick={handleClick} onContextMenu={handleRightClick} />
  }
}

export const WithSelectVideoItem = withSelect<WithSelectProps>(VideoTaskItem)
