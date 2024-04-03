import Icon from '@/components/Icon'
import classNames from 'classnames'
import {
  DropdownMenuPrimitive as DropdownMenu,
  // DropdownMenuRoot,
  // DropdownMenuContent,
  // DropdownMenuItem,
  // DropdownMenuSeparator,
  // DropdownMenuTrigger,
} from '@muse/ui/v1/dropdown-menu'
import { cn } from '@/lib/utils'
import { PropsWithChildren, ReactNode } from 'react'

export type DropdownMenuOptions =
  | {
      label: string | ReactNode
      handleClick: () => void
    }
  | 'Separator'

type _DropdownMenuProps = {
  triggerIcon?: ReactNode
  options: Array<DropdownMenuOptions>
  contentClassName?: string
}

export default function _DropdownMenu({
  options,
  triggerIcon,
  contentClassName,
  children,
}: PropsWithChildren<_DropdownMenuProps>) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        {children ? (
          children
        ) : (
          <div className={classNames(
            'inline-flex items-center justify-center size-6 rounded border border-app-line',
            'cursor-default data-[state=open]:bg-app-hover',
          )}>
            <span className="sr-only">Open menu</span>
            {triggerIcon ? triggerIcon : <Icon.more />}
          </div>
        )}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end" className={cn(
        'w-36 rounded-md text-ink bg-app-box border border-app-line p-1 shadow-lg',
        'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2',
        contentClassName,
      )}>
        {options.map((o, index) => (
          <div key={index}>
            {o === 'Separator' ? (
              <DropdownMenu.Separator className="bg-app-line h-px my-1" />
            ) : (
              <DropdownMenu.Item
                key={index}
                className={classNames(
                  'relative cursor-default select-none outline-none',
                  'focus:bg-accent focus:text-white hover:bg-accent hover:text-white',
                  'data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
                  'flex items-center justify-start rounded-md px-1 py-1 text-xs',
                )}
                onClick={o.handleClick}
              >
                {o.label}
              </DropdownMenu.Item>
            )}
          </div>
        ))}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
