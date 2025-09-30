import { Box, Button, Container, Flex, Heading, Separator, Text } from '@radix-ui/themes';
import IntentPanel from './intent/IntentPanel';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

type ActionArgSpec = { name: string; type?: string; required?: boolean };
type AppAction = { name: string; title?: string; args?: ActionArgSpec[] };
type AppDescriptor = {
    id: string;
    name?: string;
    description?: string;
    actions?: AppAction[];
    icon?: string;
};

export default function App() {
    const [apps, setApps] = useState<AppDescriptor[]>([]);

    useEffect(() => {
        (async () => {
            try {
                const list = await invoke<AppDescriptor[]>('list_apps');
                setApps(list || []);
            } catch (e) {
                console.warn('list_apps failed', e);
            }
        })();
    }, []);

    const openHosts = async () => {
        await invoke('open_with_args', { app_name: 'hostsManager' });
    };

    const runAction = async (app: AppDescriptor, action: AppAction) => {
        let argsQuery = '';
        const argSpecs = action.args || [];
        if (argSpecs.length > 0) {
            const parts: string[] = [];
            for (const spec of argSpecs) {
                const v =
                    window.prompt(`请输入参数 ${spec.name}${spec.required ? ' (必填)' : ''}`) ?? '';
                if (v || spec.required) {
                    parts.push(`${spec.name}=${encodeURIComponent(v)}`);
                }
            }
            argsQuery = parts.join('&');
        }
        await invoke('open_with_args', { appName: app.id, args: argsQuery || undefined });
    };

    return (
        <Container size="2">
            <Flex direction="column" gap="3" py="3">
                <Heading>Launcher</Heading>
                <Text color="gray">统一入口 · 命令面板 · 插件化</Text>
                <Separator size="4" my="2" />
                <Box>
                    <Button onClick={openHosts}>打开 Hosts 管理器</Button>
                </Box>
                <Separator size="2" my="2" />
                <Heading as="h3" size="3">
                    已发现的应用
                </Heading>
                <Box>
                    {apps.length === 0 ? (
                        <Text color="gray">未发现符合 tlfsuite.json 规范的应用</Text>
                    ) : (
                        <Flex direction="column" gap="3">
                            {apps.map((app) => (
                                <Box key={app.id}>
                                    <Flex align="center" gap="2">
                                        {app.icon ? (
                                            // eslint-disable-next-line @next/next/no-img-element
                                            <img
                                                src={app.icon}
                                                alt={app.name || app.id}
                                                width={24}
                                                height={24}
                                                style={{ borderRadius: 4 }}
                                            />
                                        ) : null}
                                        <Heading as="h4" size="2">
                                            {app.name || app.id}
                                        </Heading>
                                    </Flex>
                                    {app.description ? (
                                        <Text color="gray">{app.description}</Text>
                                    ) : null}
                                    <Flex gap="2" wrap="wrap" mt="2">
                                        {(app.actions || []).map((act) => (
                                            <Button
                                                key={act.name}
                                                variant="soft"
                                                onClick={() => runAction(app, act)}
                                            >
                                                {act.title || act.name}
                                            </Button>
                                        ))}
                                    </Flex>
                                </Box>
                            ))}
                        </Flex>
                    )}
                </Box>
                <Separator size="4" my="4" />
                <Heading as="h3" size="3">Intent 实验区</Heading>
                <Box>
                    <IntentPanel />
                </Box>
            </Flex>
        </Container>
    );
}
