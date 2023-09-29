using Microsoft.UI.Content;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Hosting;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using Microsoft.Win32;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Foundation.Collections;
using static System.Net.Mime.MediaTypeNames;


// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace Cappy3ds
{

    public sealed partial class MainWindow : Window
    {
        [ComImport, Guid("63aad0b8-7c24-40ff-85a8-640d944cc325"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        public interface ISwapChainPanelNative
        {
           // [PreserveSig]
           // HRESULT SetSwapChain(IDXGISwapChain swapChain);
        }

        public MainWindow()
        {
            this.InitializeComponent();

            unsafe
            {
                CsBindgen.NativeMethods.send_visual();
            }

            // renderView.setup();

            //ISwapChainPanelNative panelNative = WinRT.CastExtensions.As<ISwapChainPanelNative>(swapChainPanel1);
            //hr = panelNative.SetSwapChain(m_pDXGISwapChain1);
        }

        private void myButton_Click(object sender, RoutedEventArgs e)
        {
            myButton.Content = "Clicked";
        }
    }
}
