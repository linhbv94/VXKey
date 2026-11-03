// Public macOS APIs only. Peer identity is bound to the connected audit token,
// never to a PID that another process could reuse between lookup and validation.
#include <Security/Security.h>
#include <CoreFoundation/CoreFoundation.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <string.h>

int vx_peer_require(int fd, const char *text) {
    uid_t uid; gid_t gid;
    audit_token_t token; socklen_t length = sizeof(token);
    if (getpeereid(fd, &uid, &gid) || uid != geteuid() ||
        getsockopt(fd, 0, LOCAL_PEERTOKEN, &token, &length) || length != sizeof(token)) return 0;
    CFDataRef data = CFDataCreate(NULL, (UInt8 *)&token, sizeof(token));
    if (!data) return 0;
    const void *key = kSecGuestAttributeAudit, *value = data;
    CFDictionaryRef attrs = CFDictionaryCreate(NULL, &key, &value, 1,
        &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks);
    SecCodeRef code = NULL; SecRequirementRef requirement = NULL;
    CFStringRef string = CFStringCreateWithCString(NULL, text, kCFStringEncodingUTF8);
    int ok = attrs && string &&
        SecCodeCopyGuestWithAttributes(NULL, attrs, kSecCSDefaultFlags, &code) == errSecSuccess &&
        SecRequirementCreateWithString(string, kSecCSDefaultFlags, &requirement) == errSecSuccess &&
        SecCodeCheckValidity(code, kSecCSStrictValidate, requirement) == errSecSuccess;
    if (requirement) CFRelease(requirement);
    if (code) CFRelease(code);
    if (string) CFRelease(string);
    if (attrs) CFRelease(attrs);
    CFRelease(data);
    return ok;
}
int vx_own_team(char *output, long capacity) {
    SecCodeRef code = NULL; CFDictionaryRef info = NULL; int ok = 0;
    if (SecCodeCopySelf(kSecCSDefaultFlags, &code) == errSecSuccess &&
        SecCodeCopySigningInformation(code, kSecCSSigningInformation, &info) == errSecSuccess) {
        CFTypeRef team = CFDictionaryGetValue(info, kSecCodeInfoTeamIdentifier);
        if (team && CFGetTypeID(team) == CFStringGetTypeID())
            ok = CFStringGetCString(team, output, capacity, kCFStringEncodingUTF8);
    }
    if (info) CFRelease(info);
    if (code) CFRelease(code);
    return ok;
}
int vx_self_require(const char *text) {
    SecCodeRef code = NULL; SecRequirementRef requirement = NULL;
    CFStringRef string = CFStringCreateWithCString(NULL, text, kCFStringEncodingUTF8);
    int ok = string && SecCodeCopySelf(0, &code) == errSecSuccess &&
        SecRequirementCreateWithString(string, 0, &requirement) == errSecSuccess &&
        SecCodeCheckValidity(code, kSecCSStrictValidate, requirement) == errSecSuccess;
    if (requirement) CFRelease(requirement);
    if (code) CFRelease(code);
    if (string) CFRelease(string);
    return ok;
}
