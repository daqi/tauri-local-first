import { createRoot } from 'react-dom/client';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from './ui/alert-dialog';
import { Button } from './ui/button';

type ConfirmProps = {
    title?: string;
    description?: string;
    onOk?: () => void;
    okText?: string;
    cancelText?: string;
    onCancel?: () => void;
    children?: React.ReactNode;
    defaultOpen?: boolean;
};

export default function Confirm({
    title = 'title',
    description = 'description',
    onOk = () => {},
    okText = 'OK',
    cancelText = 'Cancel',
    onCancel = () => {},
    children = undefined,
    defaultOpen = false
}: ConfirmProps): React.JSX.Element {
    return (
        <AlertDialog defaultOpen={defaultOpen}>
            <AlertDialogTrigger asChild>
                {children ? <Button variant="destructive">{children}</Button> : null}
            </AlertDialogTrigger>
            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>{title}</AlertDialogTitle>
                    <AlertDialogDescription>{description}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogCancel onClick={onCancel}>{cancelText}</AlertDialogCancel>
                    <AlertDialogAction
                        onClick={onOk}
                        className="bg-destructive text-white hover:bg-destructive/90"
                    >
                        {okText}
                    </AlertDialogAction>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    );
}

export function confirm(options: ConfirmProps = {}): Promise<boolean> {
  return new Promise((resolve) => {
    const container = document.createElement('div');
    document.body.appendChild(container);
    const root = createRoot(container);
    const onResolve = (ok: boolean) => {
      resolve(ok);
      setTimeout(() => {
        try { root.unmount(); } catch { /* empty */ }
        container.parentElement?.removeChild(container);
      }, 0);
    };
    root.render(
      <Confirm
        title={options.title ?? '确认操作'}
        description={options.description}
        okText={options.okText}
        cancelText={options.cancelText}
        onOk={() => onResolve(true)}
        onCancel={() => onResolve(false)}
        defaultOpen
      />
    );
  });
}
