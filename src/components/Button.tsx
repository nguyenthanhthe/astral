import type { ButtonHTMLAttributes, ReactNode } from 'react';

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';
type Size = 'md' | 'sm';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  children: ReactNode;
}

export function Button({
  variant = 'secondary',
  size = 'md',
  className,
  children,
  ...rest
}: ButtonProps) {
  const classes = ['btn', `btn--${variant}`];
  if (size === 'sm') classes.push('btn--sm');
  if (className) classes.push(className);

  return (
    <button type="button" className={classes.join(' ')} {...rest}>
      {children}
    </button>
  );
}
