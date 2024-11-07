import { matchExplorerItemWithType } from '@/Explorer/pattern'
import { ExplorerItem } from '@/Explorer/types'
import { match } from 'ts-pattern'
import AudioQuickView from './Audio'
import ImageQuickView from './Image'
import RawTextQuickView from './RawText'
import VideoQuickView from './Video'
import WebPageQuickView from './WebPage'

export default function QuickViewItem({ data }: { data: ExplorerItem }) {
  return match(data)
    .with(matchExplorerItemWithType('Video'), (props) => <VideoQuickView {...props} />)
    .with(matchExplorerItemWithType('Audio'), (props) => <AudioQuickView {...props} />)
    .with(matchExplorerItemWithType('Image'), (props) => <ImageQuickView {...props} />)
    .with(matchExplorerItemWithType('RawText'), (props) => <RawTextQuickView {...props} />)
    .with(matchExplorerItemWithType('WebPage'), (props) => <WebPageQuickView {...props} />)
    .otherwise(() => <>TODO</>)
}
