'use client'
import { useShortcut } from '@/hooks/useShortcut'
import { useCallback, useState } from 'react'

export default function Page() {
  const [logs, setLogs] = useState<string[]>([])

  const callback = useCallback((shortcutName: string) => {
    const log = `Shortcut triggered: ${shortcutName} at ${new Date().toLocaleTimeString()}`
    console.log(log)
    setLogs((prev) => [...prev, log])
  }, [])

  useShortcut('delItem', (e) => callback('delItem'))
  useShortcut('showInspector', (e) => callback('showInspector'))
  useShortcut('toggleQuickPreview', (e) => callback('toggleQuickPreview'))
  useShortcut('explorerDown', (e) => callback('explorerDown'))
  useShortcut('explorerUp', (e) => callback('explorerUp'))
  useShortcut('explorerLeft', (e) => callback('explorerLeft'))
  useShortcut('explorerRight', (e) => callback('explorerRight'))

  return (
    <div>
      <h3>Shortcut Logs:</h3>
      <div className="text-ink/50 font-mono text-xs">
        {logs.map((log, index) => (
          <div key={index}>{log}</div>
        ))}
      </div>
    </div>
  )
}
