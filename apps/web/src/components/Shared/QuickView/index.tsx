import { Video } from '@/components/Video'
import { useCurrentLibrary } from '@/lib/library'
import Icon from '@gendam/ui/icons'
import Image from 'next/image'
import { useQuickViewStore, type QuickViewItem } from './store'

const Player = ({ data }: { data: QuickViewItem }) => {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="flex h-full w-full items-center justify-center">
      {data.assetObject.mimeType?.includes('video/') ? (
        <Video hash={data.assetObject.hash} />
      ) : (
        <div className="relative h-full w-full">
          <Image
            src={currentLibrary.getFileSrc(data.assetObject.hash)}
            alt={data.assetObject.hash}
            fill={true}
            className="h-full w-full rounded-md object-contain object-center"
            priority
          />
        </div>
      )}
    </div>
  )
}

export default function QuickView() {
  const quickViewStore = useQuickViewStore()

  // quickViewStore.show === true 的时候 quickViewStore.data 不会为空，这里只是为了下面 tsc 检查通过
  return quickViewStore.show && quickViewStore.data ? (
    <div className="fixed left-0 top-0 h-full w-full bg-black/50 px-20 py-10" onClick={() => quickViewStore.close()}>
      <div
        className="relative h-full w-full rounded-lg bg-black/50 px-8 pb-8 pt-20 shadow backdrop-blur-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="absolute left-0 top-6 w-full overflow-hidden px-12 text-center font-medium text-white/90">
          <div className="truncate">{quickViewStore.data.name}</div>
        </div>
        <Player data={quickViewStore.data} />
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
