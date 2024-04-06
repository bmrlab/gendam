import { Toaster } from '@/components/Toast/toaster'
import { Toaster as SonnerToaster } from '@/components/Toast/sonner'
import UploadQueue from './UploadQueue'
import QuickView from './QuickView'

export default function Shared() {
  return (
    <>
      <UploadQueue />
      <QuickView />
      <Toaster />
      <SonnerToaster />
    </>
  )
}
