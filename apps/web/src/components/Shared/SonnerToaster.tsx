'use client'
import { Toaster } from 'sonner'

type ToasterProps = React.ComponentProps<typeof Toaster>

const SonnerToaster = ({ ...props }: ToasterProps) => {
  return (
    <Toaster
      // className="group"
      // toastOptions={{
      //   classNames: {
      //     toast: 'group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg',
      //     description: 'group-[.toast]:text-muted-foreground',
      //     actionButton: 'group-[.toast]:bg-primary group-[.toast]:text-primary-foreground',
      //     cancelButton: 'group-[.toast]:bg-muted group-[.toast]:text-muted-foreground',
      //   },
      // }}
      toastOptions={{
        unstyled: true,
        classNames: {
          toast: 'bg-blue-400',
          title: 'text-red-400',
          description: 'text-red-400',
          actionButton: 'bg-zinc-400',
          cancelButton: 'bg-orange-400',
          closeButton: 'bg-lime-400',
        },
      }}
      {...props}
    />
  )
}

export default SonnerToaster
