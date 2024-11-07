import { matchRetrievalResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { match } from 'ts-pattern'
import AudioSearchItem from '../../SearchResultItem/Audio'
import ImageSearchItem from '../../SearchResultItem/Image'
import RawTextSearchItem from '../../SearchResultItem/RawText'
import VideoSearchItem from '../../SearchResultItem/Video'
import WebPageSearchItem from '../../SearchResultItem/WebPage'
import AudioRetrievalItem from './Audio'
import ImageRetrievalItem from './Image'
import RawTextRetrievalItem from './RawText'
import VideoRetrievalItem from './Video'
import WebPageRetrievalItem from './WebPage'

export default function RetrievalResultItem(props: ExtractExplorerItem<'RetrievalResult'>) {
  return match(props)
    .with(matchRetrievalResult('Video'), (props) => <VideoRetrievalItem {...props} />)
    .with(matchRetrievalResult('Audio'), (props) => <AudioRetrievalItem {...props} />)
    .with(matchRetrievalResult('Image'), (props) => <ImageRetrievalItem {...props} />)
    .with(matchRetrievalResult('RawText'), (props) => <RawTextRetrievalItem {...props} />)
    .with(matchRetrievalResult('WebPage'), (props) => <WebPageRetrievalItem {...props} />)
    .otherwise(() => <div></div>)
}

export function RetrievalResultItemPreview(props: ExtractExplorerItem<'RetrievalResult'>) {
  return match(props)
    .with(matchRetrievalResult('Video'), (props) => <VideoSearchItem {...props} />)
    .with(matchRetrievalResult('Audio'), (props) => <AudioSearchItem {...props} />)
    .with(matchRetrievalResult('Image'), (props) => <ImageSearchItem {...props} />)
    .with(matchRetrievalResult('RawText'), (props) => <RawTextSearchItem {...props} />)
    .with(matchRetrievalResult('WebPage'), (props) => <WebPageSearchItem {...props} />)
    .otherwise(() => <div></div>)
}
