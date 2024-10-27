/**
 * This file is inspired by and referenced from:
 * https://github.com/spacedriveapp/spacedrive/blob/main/interface/hooks/useShortcut.ts
 */

import { useMemo } from 'react'
import { useHotkeys } from 'react-hotkeys-hook'

type OperatingSystem = 'browser' | 'linux' | 'macOS' | 'windows' | 'unknown'
type Shortcut = Partial<Record<OperatingSystem | 'all', string[]>>

const shortcuts = {
  delItem: {
    macOS: ['meta', 'backspace'],
    all: ['delete'],
  },
  showInspector: {
    macOS: ['meta', 'i'],
    all: ['ctrl', 'i'],
  },
  toggleQuickPreview: {
    all: ['space'],
  },
  explorerDown: {
    all: ['down'],
  },
  explorerUp: {
    all: ['up'],
  },
  explorerLeft: {
    all: ['left'],
  },
  explorerRight: {
    all: ['right'],
  },
} satisfies Record<string, Shortcut>

export const useShortcut = (
  shortcutName: keyof typeof shortcuts,
  callback: (e: KeyboardEvent) => void,
  options: { disabled?: boolean } = {},
) => {
  const os: OperatingSystem = 'macOS' // TODO: Implement OS detection

  const shortcutKeys = useMemo(() => {
    const shortcut = shortcuts[shortcutName]
    const _os = os as keyof typeof shortcut
    const keys = _os in shortcut ? shortcut[_os] : shortcut['all']
    const shortcutKeys = Array.isArray(keys) ? keys.join('+') : keys
    return shortcutKeys
  }, [shortcutName, os])

  useHotkeys(
    shortcutKeys,
    (e) => {
      if (process.env.NODE_ENV !== 'development') {
        e.preventDefault()
      }
      callback(e)
    },
    {
      enabled: !options.disabled,
    },
    [callback],
  )
}
