import { useCurrentLibrary } from '@/lib/library'
import classNames from 'classnames'
import { useEffect, useState } from 'react'

export default function RawTextViewer({ hash, variant }: { hash: string; variant: 'sm' | 'default' }) {
  const currentLibrary = useCurrentLibrary()

  const [text, setText] = useState('')

  useEffect(() => {
    const textUrl = currentLibrary.getFileSrc(hash)
    fetch(textUrl).then((resp) => {
      resp.text().then(setText)
    })
  }, [hash])

  return <div className={classNames('h-full w-full p-2 overflow-scroll select-text cursor-text whitespace-pre-line', variant === 'sm' && 'text-sm')}>{text}</div>
}
