'use client'
import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from '../../public/locales/en/common.json'
import zhCN from '../../public/locales/zh-CN/common.json'

const resources = {
  en: {
    translation: en,
  },
  zhCN: {
    translation: zhCN,
  },
}

i18n.use(initReactI18next).init({
  resources,
  lng: 'en',
  interpolation: {
    escapeValue: false,
  },
})

export default i18n
