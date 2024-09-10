import { ExtractExplorerItemWithType } from '@/Explorer/types'
import { Document_Light } from '@gendam/assets/images'
import Image from 'next/image'

export default function RawTextThumb({
  data,
  className,
}: {
  data: ExtractExplorerItemWithType<'rawText'>['assetObject']
  className?: string
}) {
  return <Image src={Document_Light} alt={data.hash} fill={true} className="object-contain" priority />
}
