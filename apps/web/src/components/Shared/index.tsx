import { Toaster } from '@/components/Toast/toaster'
import { Toaster as SonnerToaster } from '@/components/Toast/sonner'
import UploadQueue from './UploadQueue'

export default function Shared() {
  return (
    <>
      <UploadQueue />
      <Toaster />
      <SonnerToaster />
    </>
  )
}
