import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function WebPageViewer({ hash }: { hash: string }) {
  const currentLibrary = useCurrentLibrary()

  // TODO 需要根据当前文件的类型来决定是否显示预览图
  return <Image src={currentLibrary.getThumbnailSrc(hash, 'webPage')} alt={hash} fill />
}
