import { useCurrentLibrary } from '@/lib/library'
import { useEffect, useState } from 'react'

export default function RawTextViewer({ hash }: { hash: string }) {
  const currentLibrary = useCurrentLibrary()

  const [text, setText] = useState('')

  useEffect(() => {
    const textUrl = currentLibrary.getFileSrc(hash)
    fetch(textUrl).then((resp) => {
      resp.text().then(setText)
    })
  }, [])

  return <div>{text}</div>
}
