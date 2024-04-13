import SonnerToaster from '@/components/Shared/SonnerToaster'
import UploadQueue from './UploadQueue'
import QuickView from './QuickView'

export default function Shared() {
  return (
    <>
      <UploadQueue />
      <QuickView />
      <SonnerToaster />
    </>
  )
}
