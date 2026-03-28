import { Test, TestingModule } from '@nestjs/testing';
import { UserController } from './user.controller';
import { UserService } from './user.service';

describe('UserController', () => {
  let controller: UserController;
  let userService: { updateUserProfile: jest.Mock };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [UserController],
      providers: [
        {
          provide: UserService,
          useValue: {
            getUserProfile: jest.fn(),
            searchUsers: jest.fn(),
            updateUserProfile: jest.fn(),
            changePassword: jest.fn(),
            updateAvatar: jest.fn(),
            deactivateUser: jest.fn(),
            getUserDetails: jest.fn(),
            getUsersByRole: jest.fn(),
            assignRole: jest.fn(),
            reactivateUser: jest.fn(),
          },
        },
      ],
    }).compile();

    controller = module.get<UserController>(UserController);
    userService = module.get(UserService);
  });

  it('patches a single profile field for the authenticated user', async () => {
    userService.updateUserProfile.mockResolvedValue({
      id: 'user-1',
      bio: 'Updated bio',
    });

    const result = await controller.update(
      { user: { userId: 'user-1' } },
      { bio: 'Updated bio' },
    );

    expect(userService.updateUserProfile).toHaveBeenCalledWith('user-1', {
      bio: 'Updated bio',
    });
    expect(result).toEqual({
      id: 'user-1',
      bio: 'Updated bio',
    });
  });
});
