import React from 'react'

export type SvgIconProps = React.SVGProps<SVGSVGElement>

export default {
  copy: (props: SvgIconProps) => (
    <svg {...props} width="16" height="16" viewBox="0 0 16 16" fill="transparent" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M13.5623 11.5397V2.43768H4.45996"
        stroke="white"
        strokeWidth="1.01132"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M11.54 4.46045H2.43774V13.5624H11.54V4.46045Z"
        stroke="white"
        strokeWidth="1.01132"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  arrowDown: (props: SvgIconProps) => (
    <svg {...props} width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
      <g>
        <path
          d="M3.66895 6.08838C3.66895 6.24951 3.72754 6.38135 3.83496 6.49365L7.5459 10.2925C7.68262 10.4243 7.82422 10.4927 8 10.4927C8.1709 10.4927 8.32227 10.4292 8.44922 10.2925L12.165 6.49365C12.2725 6.38135 12.3311 6.24951 12.3311 6.08838C12.3311 5.76123 12.0771 5.50732 11.7549 5.50732C11.5986 5.50732 11.4473 5.57568 11.335 5.68311L7.99512 9.10596L4.66504 5.68311C4.54785 5.5708 4.40625 5.50732 4.24512 5.50732C3.92285 5.50732 3.66895 5.76123 3.66895 6.08838Z"
          fill="currentColor"
        />
      </g>
    </svg>
  ),
  checked: (props: SvgIconProps) => (
    <svg {...props} width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M13.5 5.00037L6.5 12.0001L3 8.50037"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  more: (props: SvgIconProps) => (
    <svg {...props} width="17" height="16" viewBox="0 0 17 16" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M4.5625 8C4.5625 8.72487 3.97487 9.3125 3.25 9.3125C2.52513 9.3125 1.9375 8.72487 1.9375 8C1.9375 7.27513 2.52513 6.6875 3.25 6.6875C3.97487 6.6875 4.5625 7.27513 4.5625 8ZM9.8125 8C9.8125 8.72487 9.22487 9.3125 8.5 9.3125C7.77513 9.3125 7.1875 8.72487 7.1875 8C7.1875 7.27513 7.77513 6.6875 8.5 6.6875C9.22487 6.6875 9.8125 7.27513 9.8125 8ZM13.75 9.3125C14.4749 9.3125 15.0625 8.72487 15.0625 8C15.0625 7.27513 14.4749 6.6875 13.75 6.6875C13.0251 6.6875 12.4375 7.27513 12.4375 8C12.4375 8.72487 13.0251 9.3125 13.75 9.3125Z"
        fill="currentColor"
      />
    </svg>
  ),
  regenerate: (props: SvgIconProps) => (
    <svg {...props} width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M11.0105 6.73242H14.0105V3.73242" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M4.11084 4.61091C4.62156 4.10019 5.22788 3.69506 5.89517 3.41866C6.56246 3.14226 7.27766 3 7.99993 3C8.7222 3 9.4374 3.14226 10.1047 3.41866C10.772 3.69506 11.3783 4.10019 11.889 4.61091L14.0103 6.73223"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path d="M4.9895 10.2676H1.9895V13.2676" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M11.889 12.3891C11.3783 12.8999 10.772 13.305 10.1047 13.5814C9.43738 13.8578 8.72218 14.0001 7.99991 14.0001C7.27764 14.0001 6.56244 13.8578 5.89515 13.5814C5.22786 13.305 4.62154 12.8999 4.11082 12.3891L1.9895 10.2678"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  arrowUpLeft: (props: SvgIconProps) => (
    <svg width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M5 9L2 6L5 3" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M5 13H10.5C11.4283 13 12.3185 12.6313 12.9749 11.9749C13.6313 11.3185 14 10.4283 14 9.5V9.49999C14 9.04037 13.9095 8.58524 13.7336 8.1606C13.5577 7.73596 13.2999 7.35013 12.9749 7.02512C12.6499 6.70012 12.264 6.44231 11.8394 6.26642C11.4148 6.09053 10.9596 6 10.5 6H2"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
}
