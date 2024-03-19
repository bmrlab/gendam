import * as Dnd from '@dnd-kit/core'
import { PropsWithChildren } from 'react'

export const DndContext = ({ children, ...props }: PropsWithChildren<Dnd.DndContextProps>) => {
  const sensors = Dnd.useSensors(
    Dnd.useSensor(Dnd.PointerSensor, {
      // 如果不设置 activationConstraint 会覆盖点击事件导致文件夹无法被点击和双击
      activationConstraint: {
        distance: 4,
      },
    }),
  )

  return (
    <Dnd.DndContext
      {...props}
      sensors={sensors}
      collisionDetection={Dnd.pointerWithin}
      // We handle scrolling ourselves as dnd-kit
      // auto-scroll is causing issues
      autoScroll={{ enabled: false }}
    >
      {children}
    </Dnd.DndContext>
  )
}
