import Audio from '@/components/FileView/Audio'
import { PickQuickViewItem } from './store'

export default function AudioQuickView({ data }: { data: PickQuickViewItem<'Audio'> }) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Audio hash={data.assetObject.hash} duration={data.assetObject.mediaData.duration} />
    </div>
  )
}
