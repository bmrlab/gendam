import { matchSearchResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { match } from 'ts-pattern'
import AudioSearchItem from './Audio'
import ImageSearchItem from './Image'
import RawTextSearchItem from './RawText'
import VideoSearchItem from './Video'
import WebPageSearchItem from './WebPage'

export default function SearchResultItem({ data }: { data: ExtractExplorerItem<'SearchResult'> }) {
  return match(data)
    .with(matchSearchResult('video'), (props) => <VideoSearchItem {...props} />)
    .with(matchSearchResult('audio'), (props) => <AudioSearchItem {...props} />)
    .with(matchSearchResult('image'), (props) => <ImageSearchItem {...props} />)
    .with(matchSearchResult('rawText'), (props) => <RawTextSearchItem {...props} />)
    .with(matchSearchResult('webPage'), (props) => <WebPageSearchItem {...props} />)
    .otherwise(() => <></>)
}
