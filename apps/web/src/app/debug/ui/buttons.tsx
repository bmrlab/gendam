import { Button } from '@muse/ui/v2/button'

export default function Icons() {
  const buttons = []
  for (const variant of ['ghost', 'outline', 'accent', 'destructive']) {
    for (const size of ['xs', 'sm', 'md', 'lg']) {
      buttons.push(
        <div key={`${variant}-${size}`} className="w-40 flex items-end">
          <Button variant={variant as any} size={size as any}>{variant} {size}</Button>
        </div>
      )
    }
    buttons.push(<div key={variant} className='w-full'></div>)
  }
  return (
    <div className="flex flex-wrap gap-4">
      {buttons.map((button, i) => button)}
    </div>
  )
}
