'use client'
import { Button, ButtonProps } from '@muse/ui/v1/button'
import React from 'react'

interface WithOpenProps {
  onSelection?: (dir: string) => Promise<void>
}

function withDownloadDialog<T extends WithOpenProps>(Component: React.ComponentType<T>) {
  return function dialog({ ...props }: T & WithOpenProps) {
    const handleClick = async () => {
      const { onSelection } = props as WithOpenProps
      const { open } = await import('@tauri-apps/api/dialog')
      const { downloadDir } = await import('@tauri-apps/api/path')
      const selected = await open({
        multiple: false,
        directory: true,
        defaultPath: await downloadDir(),
      })

      if (onSelection && selected) {
        await onSelection(selected as string)
      }
    }

    const { onSelection, ...newProps } = props
    return <Component {...(newProps as T)} onClick={handleClick} />
  }
}

export const WithDownloadDialogButton = withDownloadDialog<ButtonProps & WithOpenProps>(Button)
