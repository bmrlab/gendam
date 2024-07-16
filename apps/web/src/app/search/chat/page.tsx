'use client'

import Viewport from '@/components/Viewport'
import { rspc } from '@/lib/rspc'
import { useState } from 'react'

export default function TestChatPage() {
  const [text, setText] = useState('')
  const [response, setResponse] = useState('')
  const [startChat, setStartChat] = useState(false)

  rspc.useSubscription(['search.chat', { text }], {
    enabled: startChat,
    onStarted: () => {
      console.log('chat started')
    },
    onData: (data) => {
      if (typeof data.response === 'undefined' || data.response === null) {
        console.log('disable subscribe to chat')
        setStartChat(false)
      } else {
        setResponse((v) => (v += data.response))
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
