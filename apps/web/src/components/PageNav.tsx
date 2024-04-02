import { PropsWithChildren } from 'react'
import Icon from '@/components/Icon'

export default function PageNav({ title }: { title?: string }) {
  return (
    <div className="flex select-none items-center">
      <div
        className="p-1 rounded-md hover:bg-toolbar-hover"
        onClick={() => window.history.back()}
      ><Icon.arrowLeft className="h-4 w-4" /></div>
      <div
        className="p-1 rounded-md hover:bg-toolbar-hover"
        onClick={() => window.history.forward()}
      ><Icon.arrowRight className="h-4 w-4" /></div>
      {title ? <div className="ml-2 text-sm">{title}</div> : null}
    </div>
  )
}
