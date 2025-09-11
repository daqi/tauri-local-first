import React from 'react';
import { Button as RxButton, type ButtonProps as RxButtonProps } from '@radix-ui/themes';

export type ButtonProps = RxButtonProps;
export const Button: React.FC<ButtonProps> = (props) => <RxButton {...props} />;
