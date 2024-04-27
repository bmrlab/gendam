'use client'
import { GenDAM_Logo } from '@gendam/assets/images'
import Image from 'next/image'
import Viewport from '@/components/Viewport'
import { Button } from '@gendam/ui/v2/button'
import { useCallback, useEffect, useState } from 'react'
import { Auth } from '@/lib/bindings'
import { client } from '@/lib/rspc'

function generateRandomString(maxLength = 10) {
  const charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  let result = '';
  for (let i = 0; i < maxLength; i++) {
    result += charset[Math.floor(Math.random() * charset.length)];
  }
  return result;
}

export default function DeviceAuth({ onSuccess }: {
  onSuccess: (auth: Auth) => void
}) {
  const [code, setCode] = useState<string|null>(null)

  const openLogin = useCallback(async () => {
    const code = generateRandomString()
    setCode(code)
    const authUrl = `https://gendam.ai/login/device?code=${code}`
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      const { open } = await import('@tauri-apps/api/shell')
      open(authUrl)
    } else {
      (window as any).open(authUrl)
    }
  }, [])

  const checkForLoginStatus = useCallback(async (code: string) => {
    const url = `https://gendam.ai/api/user/getUserByCode?code=${code}`
    try {
      const res = await fetch(url)
      const data = await res.json()
      console.log('login status', data)
      return data
    } catch (error) {
      console.info('login status', error)
    }
  }, [])

  useEffect(() => {
    if (code) {
      const interval = setInterval(async () => {
        let data: any;
        try {
          data = await checkForLoginStatus(code)
        } catch (error) {
          console.error('checkForLoginStatus failed', error)
        }
        if (!data?.user?.id) {
          return
        }
        setCode(null)
        const auth: Auth = {
          id: data.user.id,
          name: data.user.name || '',
        }
        try {
          await client.mutation(['users.set', auth])
          onSuccess(auth)
        } catch(error: any) {
          console.error('users.set failed', error)
        }
      }, 1000);
      return () => {
        clearInterval(interval);
      };
    }
  }, [checkForLoginStatus, code, onSuccess]);

  return (
    <Viewport.Page>
      <Viewport.Content>
        <div className="flex flex-col h-screen w-screen items-center justify-center">
          <div className="relative h-12 w-12">
            <Image src={GenDAM_Logo.src} alt="GenDAM App Logo" fill={true} className="object-contain" />
          </div>
          <div className="mt-4">
          Please log in to continue using <strong>GenDAM</strong>.
          </div>
          <div className="mt-16">
            <Button variant="accent" size="md" onClick={openLogin}>Log in</Button>
          </div>
        </div>
      </Viewport.Content>
    </Viewport.Page>
  )
}
