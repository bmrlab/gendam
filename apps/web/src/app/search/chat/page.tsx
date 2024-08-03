'use client'

import Viewport from '@/components/Viewport'
import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { useEffect, useMemo, useState } from 'react'
import Markdown from 'react-markdown'
import { RAGReferenceList } from './ReferenceList'

enum ResponseState {
  INIT,
  LOADING,
  FETCHING_REFERENCE,
  GENERATING,
  ERROR,
  DONE,
}

export default function TestChatPage() {
  const [text, setText] = useState('')
  const [response, setResponse] = useState('')
  const [referenceList, setReferenceList] = useState<ExtractExplorerItem<'RetrievalResult'>[]>([])
  const [errorMessage, setErrorMessage] = useState<string | undefined>(void 0)
  const [responseState, setResponseState] = useState<ResponseState>(ResponseState.INIT)

  const isChatting = useMemo(
    () => ![ResponseState.INIT, ResponseState.ERROR, ResponseState.DONE].includes(responseState),
    [responseState],
  )

  useEffect(() => {
    if (responseState === ResponseState.LOADING) {
      setResponse('')
      setErrorMessage(void 0)
      setReferenceList([])
    }
  }, [responseState])

  rspc.useSubscription(['search.rag', { query: text }], {
    enabled: isChatting,
    onStarted: () => {
      setResponseState(ResponseState.FETCHING_REFERENCE)
    },
    onData: (result) => {
      if (result.result_type === 'Reference') {
        setReferenceList((v) => [
          ...v,
          {
            type: 'RetrievalResult',
            taskType: result.data.taskType,
            assetObject: result.data.filePath.assetObject!,
            score: result.data.score,
            metadata: result.data.metadata,
          },
        ])
      } else if (result.result_type === 'Done') {
        setResponseState(ResponseState.DONE)
      } else if (result.result_type === 'Response') {
        setResponseState(ResponseState.GENERATING)
        setResponse((v) => (v += result.data))
      } else if (result.result_type === 'Error') {
        setResponseState(ResponseState.ERROR)
        setErrorMessage(result.data)
      } else {
        console.error('unknown result type')
      }
    },
    onError: (err) => {
      console.error(`chat on error: ${err}`)
    },
  })

  return (
    <Viewport.Page>
      <Viewport.Content className="z-0">
        {responseState === ResponseState.INIT && (
          <div className="flex h-full w-full flex-1 flex-col items-center justify-center p-16">
            <input
              className="border-app-line bg-app-overlay text-ink block w-full rounded-md border px-4 py-[0.3rem] pl-7 pr-7 text-sm outline-none"
              value={text}
              onChange={(e) => {
                setText(e.target.value)
              }}
              disabled={isChatting}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  setResponse('')
                  setResponseState(ResponseState.LOADING)
                }
              }}
              placeholder="Ask anything..."
            />
          </div>
        )}

        {responseState !== ResponseState.INIT && (
          <div className="flex flex-col p-8">
            {
              <div className="bg-app sticky top-0 z-10 flex w-full justify-between overflow-hidden py-4">
                <h1 className="truncate text-xl font-bold">{text}</h1>
                <Button
                  onClick={() => {
                    setResponseState(ResponseState.INIT)
                    setText('')
                  }}
                >
                  Clear
                </Button>
              </div>
            }
            <div className="mt-4 w-full">
              <RAGReferenceList items={referenceList} isLoading={responseState === ResponseState.FETCHING_REFERENCE} />
            </div>
            {response.length > 0 && (
              <div className="mt-8 flex flex-col space-y-2">
                <h2 className="text-lg font-bold">Answer</h2>
                <Markdown>{response}</Markdown>
              </div>
            )}

            {responseState !== ResponseState.GENERATING &&
              responseState !== ResponseState.DONE &&
              responseState !== ResponseState.ERROR && (
                <div className=" mt-4 flex w-full items-center justify-center">
                  <Icon.Loading className="h-5 w-5 animate-spin" />
                </div>
              )}
          </div>
        )}
      </Viewport.Content>
    </Viewport.Page>
  )
}
