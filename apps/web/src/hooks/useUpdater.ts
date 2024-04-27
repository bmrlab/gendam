import { type UpdateManifest, type UpdateStatus } from '@tauri-apps/api/updater'
import { useCallback, useEffect, useState } from 'react'
import { create } from 'zustand'

const isTauri = typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined'

const useUpdaterStore = create<{
  hasRunOnce: boolean
  checkUpdate: () => Promise<{ shouldUpdate: boolean; manifest?: UpdateManifest; }>
}>((set, get) => ({
  hasRunOnce: false,
  checkUpdate: async () => {
    const hasRunOnce = get().hasRunOnce
    if (hasRunOnce) {
      return { shouldUpdate: false }
    }
    set({ hasRunOnce: true })
    const { checkUpdate } = await import('@tauri-apps/api/updater')
    console.log('Checking for updates')
    const { shouldUpdate, manifest } = await checkUpdate()
    return {
      shouldUpdate,
      manifest
    }
  },
}))

const useVersion = () => {
  const [version, setVersion] = useState<string>('')
  useEffect(() => {
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      import('@tauri-apps/api/app').then(async ({ getVersion }) => {
        const version = await getVersion()
        setVersion(version)
      })
    }
  }, [])
  return version
}

export function useUpdater() {
  const currentVersion = useVersion()
  const { checkUpdate } = useUpdaterStore()
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>('UPTODATE')
  const [updateError, setUpdateError] = useState<string|undefined>()

  const confirmUpdate = useCallback(
    async (manifest: UpdateManifest) => {
      const { installUpdate } = await import('@tauri-apps/api/updater')
      const { confirm } = await import('@tauri-apps/api/dialog')
      const message = `GenDAM ${manifest.version} is available. You have ${currentVersion}. Would you like to update?`
      const yes = await confirm(message, {
        title: 'A new version of GenDAM is available!',
        okLabel: 'Yes',
        cancelLabel: 'No',
      })
      if (yes) {
        installUpdate()
      }
    },
    [currentVersion],
  )

  // check for updates
  useEffect(() => {
    if (!isTauri) {
      return
    }
    if (!currentVersion) {
      // 确保 confirmUpdate 里面的 currentVersion 是有值的
      return
    }
    checkUpdate().then(({ shouldUpdate, manifest }) => {
      if (shouldUpdate && manifest) {
        confirmUpdate(manifest)
      }
    })
  }, [checkUpdate, confirmUpdate, currentVersion])

  // listen to updater events
  useEffect(() => {
    if (!isTauri) {
      return
    }
    let unlisten: () => void
    let isExit = false
    import('@tauri-apps/api/updater').then(async ({ onUpdaterEvent }) => {
      unlisten = await onUpdaterEvent(({ error, status }) => {
        console.log('Updater event', error, status)
        setUpdateStatus(status)
        setUpdateError(error)
      })
      if (isExit) {
        unlisten()
      }
    })
    return () => {
      isExit = true
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  return { updateStatus, updateError, currentVersion }
}
