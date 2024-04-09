import Icons from './icons'
import Buttons from './buttons'

export default function Debug() {
  return (
    <div className="h-screen flex-1 overflow-auto p-6">
      <Icons />
      <div className='my-8'></div>
      <Buttons />
    </div>
  )
}
