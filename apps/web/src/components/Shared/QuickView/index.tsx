'use client'

import QuickViewItem from '@/components/FileContent/QuickView'
import { useCurrentLibrary } from '@/lib/library'
import Icon from '@gendam/ui/icons'
import { useQuickViewStore } from './store'

export default function QuickView() {
  const quickViewStore = useQuickViewStore()
  const currentLibrary = useCurrentLibrary()

  // quickViewStore.show === true 的时候 quickViewStore.data 不会为空，这里只是为了下面 tsc 检查通过
  return quickViewStore.show && quickViewStore.data ? (
    <div className="fixed left-0 top-0 h-full w-full bg-black/50" onClick={() => quickViewStore.close()}>
      <div
        className="relative h-full w-full overflow-hidden rounded-lg bg-black/50 px-8 pb-8 pt-20 shadow backdrop-blur-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="absolute left-0 top-6 w-full overflow-hidden px-12 text-center font-medium text-white/90">
          <div className="truncate">{quickViewStore.data.type === 'FilePath' && quickViewStore.data.filePath.name}</div>
        </div>

        <QuickViewItem data={quickViewStore.data} />
        <div
          className="absolute right-0 top-0 flex h-12 w-12 items-center justify-center p-2 hover:opacity-70"
          onClick={() => quickViewStore.close()}
        >
          <Icon.Close className="h-6 w-6 text-white/50" />
        </div>
      </div>
    </div>
  ) : (
    <></>
  )
}
