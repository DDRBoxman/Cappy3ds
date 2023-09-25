//
//  NSObject+NSViewBridge.m
//  Cappy3d
//
//  Created by Colin Edwards on 9/25/23.
//

#import "NSViewBridge.h"
#import "bindings.h"

@implementation NSViewBridge

+ (void)sendView:(nonnull NSView *)view {
    send_window((__bridge void *)(view));
}

@end
