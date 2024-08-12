import { useCurrentLibrary } from '@/lib/library'
import { BasicImageViewer } from './Image'

export default function WebPageViewer({ hash }: { hash: string }) {
  const currentLibrary = useCurrentLibrary()

  return <BasicImageViewer src={currentLibrary.getThumbnailSrc(hash, 'webPage')} alt={hash} />
}
