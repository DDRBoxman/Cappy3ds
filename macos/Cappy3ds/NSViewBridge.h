//
//  NSObject+NSViewBridge.h
//  Cappy3d
//
//  Created by Colin Edwards on 9/25/23.
//

#import <Foundation/Foundation.h>
@import AppKit;

NS_ASSUME_NONNULL_BEGIN

@interface NSViewBridge: NSObject

+ (void)sendView:(NSView*)view;

@end

NS_ASSUME_NONNULL_END
