import Buttons from './buttons'
import Forms from './forms'
import Icons from './icons'
import { P2P } from './p2p'
import Shortcuts from './shortcuts'

export default function Debug() {
  return (
    <div className="h-screen flex-1 overflow-auto p-6">
      <Shortcuts />
      <div className="my-8"></div>
      <Icons />
      <div className="my-8"></div>
      <Buttons />
      <div className="my-8"></div>
      <Forms />
      <div>p2p</div>
      <P2P />
    </div>
  )
}
