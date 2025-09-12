import React from 'react';

export type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: 'default' | 'primary' | 'danger' };
export const Button: React.FC<ButtonProps> = ({ variant = 'default', ...props }) => (
  <button
    {...props}
    style={{
      padding: '6px 12px', borderRadius: 6,
      border: variant === 'primary' ? '1px solid #2563eb' : variant === 'danger' ? '1px solid #b91c1c' : '1px solid #ddd',
      background: variant === 'primary' ? '#3b82f6' : variant === 'danger' ? '#ef4444' : '#f8f8f8',
      color: variant === 'primary' || variant === 'danger' ? '#fff' : '#111'
    }}
  />
);
