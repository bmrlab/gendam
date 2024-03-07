import Icon from '@/components/Icon'
import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { ReactNode } from 'react'

export type MuseDropdownMenuProps = {
  options: {
    label: string | ReactNode
    handleClick: () => void
  }[]
}

export default function MuseDropdownMenu({ options }: MuseDropdownMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="size-[25px] p-0 hover:bg-[#EBECEE]">
          <span className="sr-only">Open menu</span>
          <Icon.more />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="muse-border bg-[#F4F5F5] py-2 shadow-md">
        {options.map((o, index) => (
          <DropdownMenuItem
            key={index}
            className="px-2.5 py-[3.5px] text-[13px] leading-[18.2px] transition focus:bg-[#017AFF] focus:text-white data-[disabled]:text-[#AAADB2] data-[disabled]:opacity-100"
            onClick={o.handleClick}
          >
            {o.label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
