import { matchRetrievalResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { match } from 'ts-pattern'
import AudioRetrievalItem from './Audio'
import ImageRetrievalItem from './Image'
import VideoRetrievalItem from './Video'

export default function RetrievalResultItem(props: ExtractExplorerItem<'RetrievalResult'>) {
  return match(props)
    .with(matchRetrievalResult('video'), (props) => <VideoRetrievalItem {...props} />)
    .with(matchRetrievalResult('audio'), (props) => <AudioRetrievalItem {...props} />)
    .with(matchRetrievalResult('image'), (props) => <ImageRetrievalItem {...props} />)
    .otherwise(() => <div>TODO</div>)
}
