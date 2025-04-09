import { ChakraProvider } from '@chakra-ui/react';
import React from 'react';
import ReactDOM from 'react-dom/client';
import { ToastContainer } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import App from '@/App';
import { AppProvider } from '@/providers/AppProvider.tsx';
import { WalletConnectorProvider, useWalletConnector } from '@/providers/WalletConnectorProvider.tsx';
import { theme } from '@/theme';
import { deployments } from 'contracts/deployments';
import { NetworkInfo, TypinkProvider } from 'typink';

const LocalPopTestnet: NetworkInfo = {
  id: 'POP_TESTNET_LOCAL',
  name: 'Pop Testnet Local',
  logo: '',
  providers: ['ws://127.0.0.1:9944'],
  symbol: 'PAS',
  decimals: 18,
};
const DEFAULT_CALLER = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'; // Alice
const SUPPORTED_NETWORKS = [LocalPopTestnet];

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);

function TypinkApp() {
  const { wallet, connectedAccount } = useWalletConnector();

  return (
    <ChakraProvider theme={theme}>
      <TypinkProvider
        appName='Typink Dapp'
        deployments={deployments}
        defaultCaller={DEFAULT_CALLER}
        defaultNetworkId={LocalPopTestnet.id}
        supportedNetworks={SUPPORTED_NETWORKS}
        signer={wallet?.signer}
        connectedAccount={connectedAccount}>
        <AppProvider>
          <App />
          <ToastContainer
            position='top-right'
            closeOnClick
            pauseOnHover
            theme='light'
            autoClose={5_000}
            hideProgressBar
            limit={2}
          />
        </AppProvider>
      </TypinkProvider>
    </ChakraProvider>
  );
}

root.render(
  <WalletConnectorProvider>
    <TypinkApp />
  </WalletConnectorProvider>,
);
