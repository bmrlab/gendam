'use client'
import i18n from '@/lib/i18n'
import { FC, PropsWithChildren, useEffect } from 'react'
import { I18nextProvider } from 'react-i18next'

export const I18nLayout: FC<PropsWithChildren> = ({ children }) => {
  useEffect(() => {
    const lng = localStorage.getItem('lng') || 'en'
    i18n.changeLanguage(lng)
  },[])
  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
}
