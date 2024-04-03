import Icon from '@/components/Icon'
import classNames from 'classnames'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@muse/ui/v1/dropdown-menu'
import { cn } from '@/lib/utils'
import { PropsWithChildren, ReactNode } from 'react'

export type DropdownMenuOptions =
  | {
      label: string | ReactNode
      handleClick: () => void
    }
  | 'Separator'

export type MuseDropdownMenuProps = {
  triggerIcon?: ReactNode
  options: Array<DropdownMenuOptions>
  contentClassName?: string
}

export default function MuseDropdownMenu({
  options,
  triggerIcon,
  contentClassName,
  children,
}: PropsWithChildren<MuseDropdownMenuProps>) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        {children ? (
          children
        ) : (
          <div className={classNames(
            'inline-flex items-center justify-center size-6 rounded border border-app-line',
            'cursor-default data-[state=open]:bg-app-hover'
          )}>
            <span className="sr-only">Open menu</span>
            {triggerIcon ? triggerIcon : <Icon.more />}
          </div>
        )}
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className={cn('border border-[#DDDDDE] bg-[#F4F5F5] py-2 shadow-md', contentClassName)}>
        {options.map((o, index) => (
          <div key={index}>
            {o === 'Separator' ? (
              <DropdownMenuSeparator className="mx-2.5 bg-[#DDDDDE]" />
            ) : (
              <DropdownMenuItem
                key={index}
                className="px-2.5 py-[3.5px] text-[13px] leading-[18.2px] transition focus:bg-[#017AFF] focus:text-white data-[disabled]:text-[#AAADB2] data-[disabled]:opacity-100"
                onClick={o.handleClick}
              >
                {o.label}
              </DropdownMenuItem>
            )}
          </div>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
