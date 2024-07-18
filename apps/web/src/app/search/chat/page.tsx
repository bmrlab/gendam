'use client'

import Viewport from '@/components/Viewport'
import { rspc } from '@/lib/rspc'
import { useState } from 'react'

export default function TestChatPage() {
  const [text, setText] = useState('')
  const [response, setResponse] = useState('')
  const [startChat, setStartChat] = useState(false)

  rspc.useSubscription(['search.video_rag', { query: text }], {
    enabled: startChat,
    onStarted: () => {
      console.log('chat started')
    },
    onData: (data) => {
      if (data === 'Done') {
        setStartChat(false)
        console.log('RAG Done')
      } else if ('Reference' in data) {
        console.log(data.Reference)
      } else if ('Response' in data) {
        setResponse((v) => (v += data.Response))
      } else {
        console.log(data.Error)
      }
    },
    onError: (err) => {
      console.log(`chat on error: ${err}`)
    },
  })

  return (
    <Viewport.Page>
      <Viewport.Toolbar></Viewport.Toolbar>
      <Viewport.Content>
        <input
          className="border-app-line bg-app-overlay text-ink block w-full rounded-md border px-4 py-[0.3rem] pl-7 pr-7 text-sm outline-none"
          value={text}
          onChange={(e) => {
            setText(e.target.value)
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !startChat) {
              setResponse('')
              setStartChat(true)
            }
          }}
        />

        <div className="p-8 text-black">{response}</div>
      </Viewport.Content>
    </Viewport.Page>
  )
}
