import { matchRetrievalResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { match } from 'ts-pattern'
import AudioSearchItem from '../Search/Audio'
import ImageSearchItem from '../Search/Image'
import RawTextSearchItem from '../Search/RawText'
import VideoSearchItem from '../Search/Video'
import WebPageSearchItem from '../Search/WebPage'
import AudioRetrievalItem from './Audio'
import ImageRetrievalItem from './Image'
import RawTextRetrievalItem from './RawText'
import VideoRetrievalItem from './Video'
import WebPageRetrievalItem from './WebPage'

export default function RetrievalResultItem(props: ExtractExplorerItem<'RetrievalResult'>) {
  return match(props)
    .with(matchRetrievalResult('video'), (props) => <VideoRetrievalItem {...props} />)
    .with(matchRetrievalResult('audio'), (props) => <AudioRetrievalItem {...props} />)
    .with(matchRetrievalResult('image'), (props) => <ImageRetrievalItem {...props} />)
    .with(matchRetrievalResult('rawText'), (props) => <RawTextRetrievalItem {...props} />)
    .with(matchRetrievalResult('webPage'), (props) => <WebPageRetrievalItem {...props} />)
    .otherwise(() => <div></div>)
}

export function RetrievalResultItemPreview(props: ExtractExplorerItem<'RetrievalResult'>) {
  return match(props)
    .with(matchRetrievalResult('video'), (props) => <VideoSearchItem {...props} />)
    .with(matchRetrievalResult('audio'), (props) => <AudioSearchItem {...props} />)
    .with(matchRetrievalResult('image'), (props) => <ImageSearchItem {...props} />)
    .with(matchRetrievalResult('rawText'), (props) => <RawTextSearchItem {...props} />)
    .with(matchRetrievalResult('webPage'), (props) => <WebPageSearchItem {...props} />)
    .otherwise(() => <div></div>)
}
