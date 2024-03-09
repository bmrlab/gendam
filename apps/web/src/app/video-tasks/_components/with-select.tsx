'use client'

import VideoTaskItem, { VideoTaskItemProps } from './task-item'
import { VideoItem } from './task-list'
import useKeyPress, { KeyType } from '@/hooks/useKeyPress'
import { useBoundStore } from '@/store'
import React from 'react'

type WithSelectProps = {
  items: VideoItem[]
} & VideoTaskItemProps

function withSelect<T extends WithSelectProps>(Component: React.ComponentType<T>) {
  // eslint-disable-next-line react/display-name
  return function ({ ...props }: T & WithSelectProps) {
    const taskSelected = useBoundStore.use.taskSelected()
    const addTaskSelected = useBoundStore.use.addTaskSelected()
    const removeTaskSelected = useBoundStore.use.removeTaskSelected()
    const clearTaskSelected = useBoundStore.use.clearTaskSelected()

    const isShiftPressed = useKeyPress(KeyType.Shift)
    const isCommandPressed = useKeyPress(KeyType.Meta)

    const { index, videoFileHash, videoPath } = props as WithSelectProps

    const handleClick = async () => {
      // 按住 shift 键，多选视频
      if (taskSelected.length >= 1 && isShiftPressed) {
        let newTaskSelected = [
          ...taskSelected,
          {
            index,
            fileHash: videoFileHash,
            fileName: videoPath,
          },
        ]

        newTaskSelected.sort((a, b) => a.index - b.index)
        let maxIndex = newTaskSelected[newTaskSelected.length - 1].index
        let minIndex = newTaskSelected[0].index

        const needAdd = props.items.filter((item) => item.index >= minIndex && item.index <= maxIndex)
        addTaskSelected(needAdd)
        return
      }

      // 如果按住 command 键，点击已选中的视频，取消选中
      if (taskSelected.some((item) => item.videoFileHash === videoFileHash) && isCommandPressed) {
        removeTaskSelected(videoFileHash)
        return
      }

      // 默认单选视频
      clearTaskSelected()
      addTaskSelected(props)
    }

    const handleRightClick = () => {
      const notSelect = taskSelected.every((item) => item.videoFileHash !== videoFileHash)
      if (notSelect) {
        clearTaskSelected()
        addTaskSelected(props)
      }
    }

    const { index: _, items, ...newProps } = props
    return <Component {...(newProps as T)} onClick={handleClick} onContextMenu={handleRightClick} />
  }
}

export const WithSelectVideoItem = withSelect<WithSelectProps>(VideoTaskItem)
