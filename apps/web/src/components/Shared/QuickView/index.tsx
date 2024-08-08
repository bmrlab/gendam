import ImageQuickView from '@/components/FileContent/QuickView/Image'
import RawTextQuickView from '@/components/FileContent/QuickView/RawText'
import WebPageQuickView from '@/components/FileContent/QuickView/WebPage'
import { matchExplorerItemWithType } from '@/Explorer/pattern'
import Icon from '@gendam/ui/icons'
import { match } from 'ts-pattern'
import AudioQuickView from '../../FileContent/QuickView/Audio'
import VideoQuickView from '../../FileContent/QuickView/Video'
import { useQuickViewStore } from './store'

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
          <div className="truncate">{quickViewStore.data.type === 'FilePath' && quickViewStore.data.filePath.name}</div>
        </div>

        {match(quickViewStore.data)
          .with(matchExplorerItemWithType('video'), (props) => <VideoQuickView {...props} />)
          .with(matchExplorerItemWithType('audio'), (props) => <AudioQuickView {...props} />)
          .with(matchExplorerItemWithType('image'), (props) => <ImageQuickView {...props} />)
          .with(matchExplorerItemWithType('rawText'), (props) => <RawTextQuickView {...props} />)
          .with(matchExplorerItemWithType('webPage'), (props) => <WebPageQuickView {...props} />)
          .otherwise(() => (
            <>TODO</>
          ))}
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
