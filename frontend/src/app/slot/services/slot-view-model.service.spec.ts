import { TestBed } from '@angular/core/testing';

import { SlotViewModelService } from './slot-view-model.service';

describe('SlotViewModelService', () => {
  let service: SlotViewModelService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(SlotViewModelService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
