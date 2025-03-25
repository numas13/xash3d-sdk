#!/bin/sh

script=$(readlink -f "$0")
root=$(dirname "$script")

suffix="-xash"
while [[ "$#" > 0 ]]; do
    case "$1" in
        "--no-xash")
            suffix=""
            shift 1
            ;;
        *)
            break;
            ;;
    esac
done

hlsdk="$1"
if [[ ! -d "$hlsdk" ]]; then
    echo "usage: $0 [--no-xash] path/to/hlsdk"
    exit 1
fi

cd "$hlsdk" || exit 1

exec bindgen \
    --rust-target 1.64 \
    --use-core \
    --generate-cstr \
    --no-doc-comments \
    --no-layout-tests \
    --ignore-functions \
    --use-array-pointers-in-arguments \
    --allowlist-file ".*" \
    --blocklist-file "common/mathlib.h" \
    --blocklist-file "common/crc.h" \
    --blocklist-file "public/steamtypes.h" \
    --blocklist-file '/usr/.*' \
    --blocklist-item 'u?l?int\d+' \
    --blocklist-item 'u?l?intp' \
    --blocklist-item 'M_PI' \
    --blocklist-item 'cl_engsrcProxies' \
    --blocklist-item 'cl_funcs' \
    --blocklist-item 'demoapi' \
    --blocklist-item 'dlong' \
    --blocklist-item 'efx' \
    --blocklist-item 'eventapi' \
    --blocklist-item 'gEngfuncs' \
    --blocklist-item 'gEntityInterface' \
    --blocklist-item 'gGlobalVariables' \
    --blocklist-item 'gNewDLLFunctions' \
    --blocklist-item 'g_engdstAddrs' \
    --blocklist-item 'g_modfuncs' \
    --blocklist-item 'g_module' \
    --blocklist-item 'nanmask' \
    --blocklist-item 'netapi' \
    --blocklist-item 'new_cw' \
    --blocklist-item 'old_cw' \
    --blocklist-item 'pStudioAPI' \
    --blocklist-item 'pr_strings' \
    --blocklist-item 'vec3_origin' \
    --blocklist-item 'movevars' \
    --blocklist-item 'FALSE' \
    --blocklist-item 'false_' \
    --blocklist-item 'id386' \
    --blocklist-item '_S_IREAD' \
    --blocklist-item '_S_IWRITE' \
    --blocklist-item 'CRC32_t' \
    --blocklist-item 'fs_offset_t' \
    --newtype-enum 'qboolean' \
    "$root/src/wrapper$suffix.h" \
    -- \
    -I. \
    -Icommon \
    -Ipublic \
    -Ipm_shared \
    -Iengine \
    -Idlls \
    -Icl_dll
