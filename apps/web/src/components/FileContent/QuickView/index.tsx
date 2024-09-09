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
    .with(matchExplorerItemWithType('video'), (props) => <VideoQuickView {...props} />)
    .with(matchExplorerItemWithType('audio'), (props) => <AudioQuickView {...props} />)
    .with(matchExplorerItemWithType('image'), (props) => <ImageQuickView {...props} />)
    .with(matchExplorerItemWithType('rawText'), (props) => <RawTextQuickView {...props} />)
    .with(matchExplorerItemWithType('webPage'), (props) => <WebPageQuickView {...props} />)
    .otherwise(() => <>TODO</>)
}
