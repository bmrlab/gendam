import { matchSearchResult } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { match } from 'ts-pattern'
import AudioSearchItem from './Audio'
import VideoSearchItem from './Video'

export default function SearchResultItem({ data }: { data: ExtractExplorerItem<'SearchResult'> }) {
  return match(data)
    .with(matchSearchResult('video'), (props) => <VideoSearchItem {...props} />)
    .with(matchSearchResult('audio'), (props) => <AudioSearchItem {...props} />)
    .otherwise(() => <></>)
}
