#pragma once

#if defined(WIN32)
#include <d3d12.h>

bool luanda_imgui_dx12_alloc_srv(
    ID3D12Resource* resource,
    D3D12_CPU_DESCRIPTOR_HANDLE* out_cpu,
    D3D12_GPU_DESCRIPTOR_HANDLE* out_gpu);

bool luanda_imgui_dx12_write_srv(
    ID3D12Resource* resource,
    D3D12_CPU_DESCRIPTOR_HANDLE cpu);

void luanda_imgui_dx12_free_srv(
    D3D12_CPU_DESCRIPTOR_HANDLE cpu,
    D3D12_GPU_DESCRIPTOR_HANDLE gpu);
#endif
