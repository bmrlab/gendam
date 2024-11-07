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
    .with(matchSearchResult('Video'), (props) => <VideoSearchItem {...props} />)
    .with(matchSearchResult('Audio'), (props) => <AudioSearchItem {...props} />)
    .with(matchSearchResult('Image'), (props) => <ImageSearchItem {...props} />)
    .with(matchSearchResult('RawText'), (props) => <RawTextSearchItem {...props} />)
    .with(matchSearchResult('WebPage'), (props) => <WebPageSearchItem {...props} />)
    .otherwise(() => <></>)
}
