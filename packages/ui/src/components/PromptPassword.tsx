import { useRef } from 'react';
import { createRoot } from 'react-dom/client';
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';

type PromptPasswordProps = {
    title?: string;
    description?: string;
    onOk?: (str: string) => void;
    okText?: string;
    cancelText?: string;
    onCancel?: () => void;
    children?: React.ReactNode;
    defaultOpen?: boolean;
};

export default function PromptPassword({
    title = 'title',
    description = 'description',
    onOk = () => {},
    okText = 'OK',
    cancelText = 'Cancel',
    onCancel = () => {},
    children = undefined,
    defaultOpen = false,
}: PromptPasswordProps): React.JSX.Element {
    const valueRef = useRef<string>('');
    return (
        <Dialog defaultOpen={defaultOpen}>
            {children ? (
                <DialogTrigger asChild>
                    <Button variant="outline">{children}</Button>
                </DialogTrigger>
            ) : null}
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>{title}</DialogTitle>
                    <DialogDescription>{description}</DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                        <Input
                            id="password"
                            type="password"
                            defaultValue=""
                            className="col-span-4"
                            onChange={(e) => (valueRef.current = e.target.value)}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <DialogClose asChild>
                        <Button type="button" variant="outline" onClick={onCancel}>
                            {cancelText}
                        </Button>
                    </DialogClose>
                    <Button type="submit" onClick={() => onOk(valueRef.current)}>
                        {okText}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

export function promptPassword(options: PromptPasswordProps = {}): Promise<string> {
    return new Promise((resolve) => {
        const container = document.createElement('div');
        document.body.appendChild(container);
        const root = createRoot(container);
        const onResolve = (str: string) => {
            resolve(str);
            setTimeout(() => {
                try {
                    root.unmount();
                } catch {
                    /* empty */
                }
                container.parentElement?.removeChild(container);
            }, 0);
        };
        root.render(
            <PromptPassword
                title={options.title ?? '确认操作'}
                description={options.description}
                okText={options.okText}
                cancelText={options.cancelText}
                onOk={(str) => onResolve(str)}
                onCancel={() => onResolve('')}
                defaultOpen
            />,
        );
    });
}
