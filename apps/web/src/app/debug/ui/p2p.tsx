'use client'

import { useP2PEvents } from '@/hooks/useP2PEvents'
import { rspc } from '@/lib/rspc'
import { Button } from '@muse/ui/v2/button'

export const P2P = () => {
  const { data, refetch } = rspc.useQuery(['p2p.state'])
  useP2PEvents()
  return (
    <>
      <Button onClick={() => refetch()}>refetch</Button>
      <div className="text-sm">{JSON.stringify(data)}</div>
    </>
  )
}
