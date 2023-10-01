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
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Foundation.Collections;
using WinRT;



// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace Cappy3ds
{

    public sealed partial class MainWindow : Window
    {
        /// <summary>
        /// Interface from microsoft.ui.xaml.media.dxinterop.h
        /// </summary>
        [ComImport]
        [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        [Guid("63aad0b8-7c24-40ff-85a8-640d944cc325")]
        public interface ISwapChainPanelNative
        {
            [PreserveSig] uint SetSwapChain([In] IntPtr swapChain);
        }

        public MainWindow()
        {
            this.InitializeComponent();

            unsafe
            {

          
                CsBindgen.NativeMethods.send_visual((void*)((IWinRTObject)swapChainPanel1).NativeObject.GetRef(), &Sum);
            }

            // renderView.setup();


        }

        private void myButton_Click(object sender, RoutedEventArgs e)
        {
            myButton.Content = "Clicked";
        }

        [UnmanagedCallersOnly(CallConvs = new[] { typeof(CallConvCdecl) })]
        unsafe static int Sum(void* panel, void* test) {
            Debug.WriteLine("test");

            // cast panel back
            var panel2 = SwapChainPanel.FromAbi((IntPtr)panel);
            ISwapChainPanelNative panelNative = WinRT.CastExtensions.As<ISwapChainPanelNative>(panel2);

            panelNative.SetSwapChain((IntPtr)test);



            return 1;
        }
    }


}
