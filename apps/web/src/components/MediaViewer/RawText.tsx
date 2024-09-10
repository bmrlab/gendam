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
  }, [hash, currentLibrary])

  return (
    <div
      className={classNames(
        'h-full w-full cursor-text select-text overflow-scroll whitespace-pre-line p-2',
        variant === 'sm' && 'text-sm',
      )}
    >
      {text}
    </div>
  )
}
